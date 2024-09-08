# Function calling convention for the Oxide compiler

Deciding where to place function arguments:

let arg_locations = []

for arg in args.rev():
  if arg.size > 8 || registers are unavailable:
    arg_locations.push(STACK)
  else:
    arg_locations.push(REGISTER)
    save current register state (push r(n))
    update the tn location map??????


Calling a function:

1. Set up the return value store
if sizeof(return value) > 8:
  make space on the stack
else:
  the value is returned in r1

3. construct a mapper that maps each argument to a storage location.

2. Save the state of the registers that will be used to pass function arguments.
This must be done before loading any function arguments. Because of this, a structure that maps each arg to a storage location must be constructed first.

3. Function arguments are loaded in reverse order.
The function arguments are loaded
If the size of the argument is <= 8 and there are available registers, load the argument in the first available register.
If the size of the argument is > 8 or there are no available registers, push the argument on the stack.
Registers that can be used to pass function arguments are r3, r4, r5, r6, r6, r8

4. Call the function
push the return address
jump to the function start label

Cleaning up

5. pop the function arguments from the stack.

6. Restore the previous register states

7. Store the return value wherever it needs to be stored



Function being called:

1. Save the previous stack frame and initialize the new one
push sbp
mov sbp stp

2. make space for local variables
pushsp sizeof(stack frame)

Returning:

3. Store the return value either in r1 or on the stack

4. Pop the current stack frame and restore the old stack frame
mov stp sbp
pop8 sbp

5. return from the function
pop the first 8 bytes from the stack and jump to that address


Stack when a function is called

Caller's responsibility
[return value]
[arg3]
[arg2]
[arg1]
[return address]
<-- Now the it's the callee's responsibility
[old stack frame]
[local variables]

With arguments passed through registers:

[arg4 with size > 8]
[return address]

{
  r1: return value will be stored here
  r2: some random value
  r3: arg3
  r4: arg2
  r5: arg1
}

[old stack frame]
[local variables]




ORRR

For now, just pass the arguments and the return value on the stack.
Later optimizations will take care of using registers to improve performance.
Also, bear in mind that the rusty vm does not behave like an x86 machine
and specific x86 optimizations may not apply to this vm.

So, without any register optimizations:

Caller:

1. Make space for the return value, if any
pushsp sizeof(return value)

2. Load the argments on the stack in reverse order
for arg in args.rev():
  push arg

3. Call the function

4. Cleanup by popping the arguments
popsp sizeof(args)

Callee:

1. Save previous stack frame and initialize the new one
push sbp
mov sbp stp

2. Make space for local symbols
pushsp sizeof(stack frame)

... Do stuff ...

3. Restore the previous stack frame
mov stp sbp
pop8 sbp

4. Return to the caller
return
