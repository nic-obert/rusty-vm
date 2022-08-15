CC = g++
STD = c++17

SRC_DIR = src
HEADER_DIR = headers
TARGET_DIR = target


SOURCES = $(shell find $(SRC_DIR) -type f -name "*.cpp")
HEADERS = $(shell find $(HEADHER_DIR) -type f -name "*.hh")
TARGET = $(TARGET_DIR)/vm


C_FLAGS=-std=$(STD) -I $(HEADER_DIR)
WARNINGS=-Wall -Wextra -Werror


all: $(TARGET)


$(TARGET_DIR):
	mkdir -p $(TARGET_DIR)


$(TARGET): $(SOURCES) $(HEADERS) $(TARGET_DIR)
	time -f "\nTime elapsed: %E seconds" $(CC) $(C_FLAGS) $(WARNINGS) $(SOURCES) -O3 -o $(TARGET)


builddb: $(SOURCES) $(HEADERS) $(TARGET_DIR)
	time -f "\nTime elapsed: %E seconds" $(CC) -g $(C_FLAGS) $(WARNINGS) $(SOURCES) -o $(TARGET)


clean:
	rm -f $(TARGET)


count:
	find . -name '*.cpp' -o -name '*.hh' | xargs wc -l | sort -nr

