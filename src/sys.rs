use std::collections::HashMap;
use sysinfo::{System, SystemExt, ProcessExt, Process};
use crate::Scoop;

#[derive(Debug)]
pub struct SysTool(System);

impl SysTool {
  pub fn new() -> SysTool {
    SysTool(System::default())
  }

  pub fn running_apps(&mut self, scoop: &Scoop) -> HashMap<&usize, &Process> {
    // Find all running processes of installed Scoop apps.
    let root_path = scoop.config.get("root_path").unwrap().as_str().unwrap();
    self.0.refresh_processes();
    let processes = self.0.get_processes()
      .iter()
      .filter(|(_, p)| p.exe().starts_with(root_path))
      .collect::<HashMap<_, _>>();
    processes
  }
}