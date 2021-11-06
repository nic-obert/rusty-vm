

class Memory:

    def __init__(self, size: int) -> None:
        self.size = size
        self.memory = [0] * size
    

    def get_data(self, address: int, size: int) -> list:
        """
        Get data from memory.
        """
        return self.memory[address:address + size]


    def store_data(self, address: int, data: list) -> None:
        """
        Store data to memory.
        """
        self.memory[address:address + len(data)] = data

