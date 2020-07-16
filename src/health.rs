/*#![allow(non_snake_case)]
// TODO: fix, remove ^

// NOTES:
// xml serialization was dumb
// so I wrote a string instead, who cares

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleHealth {
    pub State: String, // ENUM
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Role {
    pub InstanceId: String,
    pub Health: RoleHealth,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleInstanceList {
    #[serde(rename = "Role", default)]
    pub items: Vec<Role>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Container {
    pub ContainerId: String,
    pub RoleInstanceList: RoleInstanceList,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Health {
    pub GoalStateIncarnation: i64,
    pub Container: Container,
}
*/

//const HEALTH_REPORT_TEMPALTE: &str =

pub fn mk_health_report(
    container_id: &str,
    instance_id: &str,
) -> Result<String, serde_xml_rs::Error> {
    let goal_state_incarnation = 1;
    let state = "Ready";

    let health_report_xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<Health xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <GoalStateIncarnation>{}</GoalStateIncarnation>
  <Container>
    <ContainerId>{}</ContainerId>
    <RoleInstanceList>
      <Role>
        <InstanceId>{}</InstanceId>
        <Health>
          <State>{}</State>
        </Health>
      </Role>
    </RoleInstanceList>
  </Container>
</Health>
"#,
        goal_state_incarnation, container_id, instance_id, state
    );
    Ok(health_report_xml)
    /*
    let health_report = crate::health::Health {
        GoalStateIncarnation: 1,
        Container: crate::health::Container {
            ContainerId: container_id.to_string(),
            RoleInstanceList: crate::health::RoleInstanceList {
                items: vec![crate::health::Role {
                    InstanceId: instance_id.to_string(),
                    Health: crate::health::RoleHealth {
                        State: "Ready".to_string(),
                    },
                }],
            },
        },
    };
    let health_report_xml = serde_xml_rs::to_string(&health_report)?;
    Ok(health_report_xml)
    */
}
