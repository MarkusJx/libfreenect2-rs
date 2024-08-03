use crate::types::config::Config;

#[test]
fn test_create_config() {
  assert!(Config::new().is_ok());
}

#[test]
fn test_set_min_depth() {
  let mut config = Config::new().unwrap();
  assert!(config.set_min_depth(0.0).is_err());
  config.set_max_depth(2.0).unwrap();
  assert!(config.set_min_depth(3.0).is_err());
  assert!(config.set_min_depth(1.0).is_ok());

  assert_eq!(config.get_min_depth(), 1.0);
}

#[test]
fn test_set_max_depth() {
  let mut config = Config::new().unwrap();
  assert!(config.set_max_depth(0.0).is_err());
  config.set_min_depth(0.7).unwrap();
  assert!(config.set_max_depth(0.5).is_err());
  assert!(config.set_max_depth(1.0).is_ok());

  assert_eq!(config.get_max_depth(), 1.0);
}

#[test]
fn test_set_enable_bilateral_filter() {
  let mut config = Config::new().unwrap();
  assert!(config.set_enable_bilateral_filter(true).is_ok());
  assert!(config.get_enable_bilateral_filter());
  assert!(config.set_enable_bilateral_filter(false).is_ok());
  assert!(!config.get_enable_bilateral_filter());
}

#[test]
fn test_set_enable_edge_aware_filter() {
  let mut config = Config::new().unwrap();
  assert!(config.set_enable_edge_aware_filter(true).is_ok());
  assert!(config.get_enable_edge_aware_filter());
  assert!(config.set_enable_edge_aware_filter(false).is_ok());
  assert!(!config.get_enable_edge_aware_filter());
}
