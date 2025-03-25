# Debugger TODO

- (UI) separators to resize the UI areas
- implement breakpoints (temporary breakpoints are done, persistent breakpoints are still to be implemented)
- implement DWARF-like format
- the debugger should periodically check the VM process to see if a breakpoint has been triggered. Failure to access shared memory may mean that the VM process was terminated.
- add column indices 0-15 in memory view area
