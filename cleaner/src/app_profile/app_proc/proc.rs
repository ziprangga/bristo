#[derive(Debug, Default, Clone)]
pub struct Proc {
    pid: i32,
    command: String,
    name: String,
}

impl Proc {
    /// Contruct Proc
    pub fn new(pid: i32, command: String, name: String) -> Self {
        Self { pid, command, name }
    }
    /// get the copy of pid
    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// get the reference of command
    pub fn as_command(&self) -> &str {
        &self.command
    }

    /// get the reference of process name
    pub fn as_name(&self) -> &str {
        &self.name
    }
}
