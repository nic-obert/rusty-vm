# Debugger TODO

- Fix debugger window freezing when exiting and the VM process is already terminated
- (UI) separators to resize the UI areas
- implement breakpoints
- implement DWARF-like format
- disable some actions in the UI when the VM is running. In the backend, panic if these actions are performed while VM is running
- the debugger should periodically check the VM process to see if a breakpoint has been triggered. Failure to access shared memory may mean that the VM process was terminated.
