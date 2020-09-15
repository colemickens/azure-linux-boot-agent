use thiserror::Error;

const WIRESERVER_PREFIX: &str = "http://168.63.129.16/";
const IMDS_PREFIX: &str = "http://169.254.169.254/metadata/";

const AGENT_STATE_DIR: &str = "/var/lib/azure-linux-boot-agent/";
pub const VMID_FILENAME: &str = "vmid";
pub const OVFENV_FILENAME: &str = "ovf-env.xml";

pub struct Agent {
    http_client: reqwest::Client,
    instance_id: String,
    container_id: String,
    state_dir: std::path::PathBuf,
}

#[derive(PartialEq, Debug)]
pub enum AgentMetadataMode {
    NoOp,
    Apply,
    Stash,
}

// TODO: canonical way to auto-convert from the CLI args? or just re-use struct?
// TODO: these *could* just be args to provision()...
pub struct AgentOptions {
    pub metadata_mode: AgentMetadataMode,
    pub seed_entropy: bool,
    pub create_users: bool,
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("provisioning is not needed")]
    ProvisionNotNeeded,

    #[error("user-supplied custom data failed to apply")]
    ProvisionFailedBadCustomData,

    #[error("network error")]
    ProvisionFailedNetworkError(#[from] reqwest::Error),

    #[error("the agent experienced an internal error")]
    ProvisionFailedInternalError,

    #[error("the agent experienced an IO error")]
    ProvisionFailedIOError(#[from] std::io::Error),
}

impl Agent {
    pub async fn new() -> Result<Agent, reqwest::Error> {
        let client = reqwest::Client::new();

        let mut agent = Agent {
            http_client: client,
            instance_id: String::new(),
            container_id: String::new(),
            state_dir: std::path::Path::new(&AGENT_STATE_DIR).to_path_buf(),
        };

        trace!("agent init");
        agent.init().await.expect("failed to initialize");

        info!("container_id={}", &agent.container_id);
        info!("instance_id={}", &agent.instance_id);

        //trace!("create_dir_all on {}", &AGENT_STATE_DIR);
        trace!("create_dir_all on {:?}", &agent.state_dir);
        tokio::fs::create_dir_all(&agent.state_dir)
            .await
            .expect(&format!("failed to create {}", &AGENT_STATE_DIR));

        Ok(agent)
    }

    async fn init(&mut self) -> Result<(), reqwest::Error> {
        let contents = self.wireserver_text("machine?comp=goalstate").await?;
        let goalstate: crate::goalstate::GoalState =
            serde_xml_rs::from_str(&contents).expect("failed to decode goalstate xml");
        self.container_id = goalstate.Container.ContainerId;
        self.instance_id = goalstate
            .Container
            .RoleInstanceList
            .RoleInstance
            .InstanceId
            .clone();
        Ok(())
    }

    // TODO: make this error instead of return false
    async fn report_ready(&mut self) -> Result<bool, reqwest::Error> {
        let payload = crate::health::mk_health_report(&self.container_id, &self.instance_id);
        let payload = payload.expect("failed to make health report");

        let mut url = reqwest::Url::parse(WIRESERVER_PREFIX).unwrap();
        url = url
            .join("/machine?comp=health")
            .expect("failed to build ws url");

        let req = self
            .http_client
            .post(url.as_str())
            .header("x-ms-version", "2012-11-30")
            .header("Content-Type", "text/xml;charset=utf-8")
            .header("x-ms-agent-name", "agent-linux-boot-agent")
            .body(payload.clone());
        trace!("provision report_ready request: {:?}", req);
        trace!("provision report_ready payload: {}", &payload);

        let res = req.send().await?;
        let status = res.status(); // TODO handle status
        let resptext = res.text().await.unwrap();
        trace!("provision report_ready response: {} {}", status, resptext);

        Ok(status.is_success() == true)
    }

    async fn wireserver_text(&mut self, path: &str) -> Result<String, reqwest::Error> {
        // TODO: better url concat
        let url = reqwest::Url::parse(WIRESERVER_PREFIX).unwrap();
        let url = url.join(&path).expect("failed to build ws url");
        let mut req = self.http_client.get(url.as_str());
        req = req.header("x-ms-version", "2012-11-30");
        trace!("wireserver_text req {:?}", req);
        let res = req.send().await?;
        res.text().await
    }

    async fn metadata_text(&self, path: &str) -> Result<String, reqwest::Error> {
        // TODO: better url concat
        let url = reqwest::Url::parse(IMDS_PREFIX).unwrap();
        let url = url.join(&path).expect("failed to build imdb url");
        let mut req = self.http_client.get(url.as_str());
        req = req.query(&[("api-version", "2019-06-01"), ("format", "text")]);
        req = req.header("Metadata", "true");
        trace!("metadata request: {:?}", req);
        let res = req.send().await?;
        res.text().await
    }

    async fn update_hostname(&self) -> Result<std::process::ExitStatus, reqwest::Error> {
        let hostname = self.metadata_text("instance/compute/name").await?;
        trace!("calling hostname {}", &hostname);
        let mut cmd = tokio::process::Command::new("hostname");
        let cmd = cmd.args(&[&hostname]);
        let cmd = cmd.spawn().expect("failed to start hostname");
        let status = cmd.await.expect("running hostname failed");

        match tokio::fs::write("/etc/hostname", &hostname).await {
            Ok(_) => { return Ok(status); }
            Err(e) => { error!("hostname call failed"); return Ok(status); }
        }
    }

    // TODO: is this the same as reading it from /sys/devices/dmi/...? (from udev rule)
    pub async fn get_advertised_vmid(&mut self) -> Result<String, reqwest::Error> {
        self.metadata_text("instance/compute/vmId").await
    }

    pub async fn fetch_ssh_keys(
        &mut self,
    ) -> Result<Vec<crate::metadata::instance::compute::PublicKey>, reqwest::Error> {
        let url = reqwest::Url::parse(IMDS_PREFIX).unwrap();
        let url = url
            .join("instance/compute/publicKeys")
            .expect("failed to build imdb pubkey url");
        let mut req = self.http_client.get(url.as_str());
        req = req
            .query(&[("api-version", "2019-06-01")])
            .header("Metadata", "true");
        trace!("metadata request: {:?}", req);
        let res = req.send().await?;
        res.json().await
    }

    async fn read_file(&mut self, filename: &str) -> Result<String, std::io::Error> {
        let f = self.state_dir.join(&filename);
        tokio::fs::read_to_string(&f).await
    }

    async fn write_file(&mut self, filename: &str, contents: &str) -> Result<(), std::io::Error> {
        let f = self.state_dir.join(&filename);
        tokio::fs::write(&f, contents).await
    }

    pub async fn provision(&mut self, opts: &AgentOptions) -> Result<(), AgentError> {
        trace!("in provision");
        let current_vmid: String = self
            .read_file(&VMID_FILENAME)
            .await
            .unwrap_or("".to_string());
        let advertised_vmid: String = self.get_advertised_vmid().await?;

        if current_vmid == advertised_vmid {
            return Err(AgentError::ProvisionNotNeeded);
        }

        trace!("provision wanted");

        self.update_hostname().await?;

        let ovfenv_contents = crate::osutil::read_ovf_env().await?;
        self.write_file("ovf-env.xml", &ovfenv_contents).await?;

        let customdata = self.metadata_text("instance/compute/customData").await?;

        match opts.metadata_mode {
            AgentMetadataMode::NoOp => {}
            AgentMetadataMode::Stash => {
                info!("stashing customdata");

                self.write_file("customData.raw", &customdata).await?;
            }
            AgentMetadataMode::Apply => {
                crate::nixoshelper::handle_metadata(&customdata).await.unwrap();
            }
        };

        self.update_hostname().await.expect("failed to provision");

        if opts.seed_entropy {
            info!("step_seed_entropy executing");
            crate::osutil::seed_entropy()
                .await
                .expect("failed to seed entropy");
        } else {
            info!("step_seed_entropy skipped");
        }

        if opts.create_users {
            info!("step_create_users executing");

            // TODO: consider implementing this with ovf-env.xml, except, you know, fuck that
            let keys = self.fetch_ssh_keys().await?;

            for k in keys {
                crate::osutil::create_users_with_keys(&k).await?;
            }
        } else {
            info!("step_create_users skipped");
        }

        if !self.report_ready().await.expect("failed to report ready") {
            error!("also failed to report ready");
            std::process::exit(-1); // TODO: error handling is all fucked up
        }

        self.write_file(VMID_FILENAME, &advertised_vmid).await?;

        Ok(())
    }
}
