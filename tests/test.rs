// #[test]
// fn mk_health_report() {
//     let actual_xml = azbla::health::mk_health_report(
//         "6f2b8f64-f35c-4f84-a1b0-7e71ff8e454e",
//         "aa166a1dc9fb47d8befbb43b6849ce61.lars-test-1",
//     )
//     .expect("fail");

//     let expected_xml = r#"<Health xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema"><GoalStateIncarnation>1</GoalStateIncarnation><Container>  <ContainerId>6f2b8f64-f35c-4f84-a1b0-7e71ff8e454e</ContainerId>  <RoleInstanceList>    <Role>      <InstanceId>aa166a1dc9fb47d8befbb43b6849ce61.lars-test-1</InstanceId><Health><State>Ready</State></Health></Role></RoleInstanceList></Container></Health>"#;

//     assert_eq!(actual_xml, expected_xml);
// }

use azure_linux_boot_agent as azlba;

#[test]
fn test_get_username_from_keypath() {
    let keypath = "/home/cole/.ssh/authorized_keys";
    let username = azlba::osutil::get_username_from_keypath(&keypath).unwrap();
    assert_eq!(&username, "cole");

    let keypath = "/home/cole123/.ssh/authorized_keys";
    let username = azlba::osutil::get_username_from_keypath(&keypath).unwrap();
    assert_eq!(&username, "cole123");
}
