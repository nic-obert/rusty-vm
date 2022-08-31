#pragma once

#include <unordered_map>
#include <vector>
#include <iostream>


namespace argparser
{

    // Type of the parameter
    typedef enum class TypeName : unsigned char
    {
        BOOL_EXPLICIT,
        BOOL_IMPLICIT,
        INTEGER,
        STRING,
        BOOL_POSITIONAL,
        INTEGER_POSITIONAL,
        STRING_POSITIONAL
    } TypeName;


    typedef struct Parameter
    {
        TypeName type;
        void* store;
        bool required;
        std::string description;

        Parameter();
        Parameter(TypeName type, void* store, bool required, std::string&& description);

    } Parameter;


    class Parser
    {
    private:

        std::vector<Parameter> positionals;

        std::unordered_map<std::string, Parameter> flagsMap;

        std::string description;

        void printHelp() const;

    public:

        // pre-allocates argNumber spaces for the parameters
        Parser(size_t argNumber = 1, std::string&& description = "");

        // adds an explicit boolean parameter
        // its value must be set explicitely
        void addBoolExplicit(std::string&& flagName, bool* store, bool required = false, std::string&& description = "");

        void addBoolPositional(bool* store, bool required = false, std::string&& description = "");

        // adds an implicit boolean parameter
        // its value is false if it's not passed as an argument
        // its value is true if it's passed as an argument
        void addBoolImplicit(std::string&& flagName, bool* store, bool required = false, std::string&& description = "");

        // adds an explicit integer parameter
        // its value must be set explicitely
        void addInteger(std::string&& flagName, int* store, bool required = false, std::string&& description = "");

        void addIntegerPositional(int* store, bool required = false, std::string&& description = "");

        // adds an explicit string parameter
        // its value must be set explicitely
        void addString(std::string&& flagName, const char** store, bool required = false, std::string&& description = "");

        void addStringPositional(const char** store, bool required = false, std::string&& description = "");

        // parse the command line arguments
        // set the added flags based on provided arguments
        void parse(unsigned int argc, const char** argv);

    };
 
};


std::ostream& operator<<(std::ostream& stream, argparser::TypeName typeName);

std::ostream& operator<<(std::ostream& stream, const argparser::Parameter& parameter);

