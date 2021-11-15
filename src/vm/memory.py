

class Memory:

    def __init__(self, size: int) -> None:
        self.size = size
        self.memory = [0] * size
    

    def get_data(self, address: int, size: int) -> int:
        """
        Get data from memory.
        """
        data = self.memory[address:address + size]
        return int.from_bytes(data, byteorder='big', signed=False)


    def store_data(self, address: int, data: int) -> None:
        """
        Store data to memory.
        """
        
        self.memory[address:address + len(data)] = data

