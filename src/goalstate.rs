#![allow(non_snake_case)]
// TODO: fix, remove ^

// TODO: consider removing this and using xpath or something to pull what we need

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleInstanceConfiguration {
    pub HostingEnvironmentConfig: String,
    pub SharedConfig: String,
    pub ExtensionsConfig: String,
    pub FullConfig: String,
    pub Certificates: String,
    pub ConfigName: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleInstance {
    pub InstanceId: String,
    pub State: String,
    pub Configuration: RoleInstanceConfiguration,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LBPort {
    pub Port: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleInstanceList {
    pub RoleInstance: RoleInstance,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Container {
    pub ContainerId: String, //TODO: uuid type
    pub RoleInstanceList: RoleInstanceList,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Machine {
    pub ExpectedState: String, // ENUM?
    pub StopRolesDeadlineHint: i64,
    pub LBProbePorts: Vec<LBPort>,
    pub ExpectHealthReport: String, //Enum?
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GoalState {
    pub Version: String,
    pub Incarnation: i64,
    pub Machine: Machine,
    pub Container: Container,
}
