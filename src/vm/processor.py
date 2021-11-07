from memory import Memory


class Processor:

    def __init__(self, memory_size: int) -> None:
        self.memory = Memory(memory_size)
    

    def execute(self, byte_code: bytes) -> None:
        """
        Execute the byte code.
        """
        pass

