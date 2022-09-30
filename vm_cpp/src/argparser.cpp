#include "argparser.hh"
#include <string.h>


using namespace argparser;


static inline void checkArgBounds(unsigned int argc, unsigned int i, const char *caller) {
    if (argc == i) {
        std::cerr << "Missing argument for parameter " << caller
                  << "\nUse the --help flag to show usage" << std::endl;
        exit(EXIT_FAILURE);
    }
}


Parameter::Parameter() {

}


Parameter::Parameter(TypeName type, void *store, bool required, std::string &&description)
: type(type), store(store), required(required), description(std::move(description)) 
{

}


Parser::Parser(size_t argNumber, std::string &&description)
:   positionals(std::vector<Parameter>()),
    flagsMap(std::unordered_map<std::string, Parameter>()),
    description(std::move(description))
{
    positionals.reserve(argNumber);
}


void Parser::addBoolExplicit(std::string &&flagName, bool *store, bool required, std::string &&description) {
    flagsMap[flagName] = Parameter(
        TypeName::BOOL_EXPLICIT,
        store,
        required,
        std::move(description));
}


void Parser::addBoolImplicit(std::string &&flagName, bool *store, bool required, std::string &&description) {
    flagsMap[flagName] = Parameter(
        TypeName::BOOL_IMPLICIT,
        store,
        required,
        std::move(description));

    // defaults to false, true if argument is found
    *store = false;
}


void Parser::addBoolPositional(bool *store, bool required, std::string &&description) {
    positionals.emplace_back(Parameter(
        TypeName::BOOL_POSITIONAL,
        store,
        required,
        std::move(description)));
}


void Parser::addInteger(std::string &&flagName, int *store, bool required, std::string &&description) {
    flagsMap[flagName] = Parameter(
        TypeName::INTEGER,
        store,
        required,
        std::move(description));
}


void Parser::addIntegerPositional(int *store, bool required, std::string &&description) {
    positionals.emplace_back(Parameter(
        TypeName::INTEGER_POSITIONAL,
        store,
        required,
        std::move(description)));
}


void Parser::addString(std::string &&flagName, const char **store, bool required, std::string &&description) {
    flagsMap[flagName] = Parameter(
        TypeName::STRING,
        store,
        required,
        std::move(description));
}


void Parser::addStringPositional(const char **store, bool required, std::string &&description) {
    positionals.emplace_back(Parameter(
        TypeName::STRING_POSITIONAL,
        store,
        required,
        std::move(description)));
}


void Parser::printHelp() const {
    std::cout << description << "\n\n";

    std::cout << "Positional arguments:\n";

    for (const Parameter &parameter : positionals) {
        std::cout << parameter << '\n';
    }

    std::cout << "\nKeyword arguments:\n";

    for (auto it = flagsMap.cbegin(); it != flagsMap.cend(); it++) {
        std::cout << it->first << '\t' << it->second << '\n';
    }

    std::cout << std::endl;
}


static void printArgs(unsigned int argc, const char **argv) {
    std::cout << "Provided arguments: {";

    for (unsigned int i = 0; i != argc; i++) {
        std::cout << argv[i] << ',';
    }

    std::cout << '}' << std::endl;
}


void Parser::parse(unsigned int argc, const char **argv) {
    // Current positional argument
    auto positional = positionals.begin();

    for (unsigned int i = 1; i != argc; i++) {

        // Handle eventual help command first and then exit
        if (!strcmp(argv[i], "--help")) {
            printHelp();
            exit(EXIT_SUCCESS);
        }

        TypeName type;
        void *output;

        auto it = flagsMap.find(argv[i]);
        if (it == flagsMap.end()) {
            if (argv[i][0] != '-') {
                type = positional->type;
                output = positional->store;
                // set the required bool to false even if it was not required in
                // the first place
                positional->required = false;
                // increment current positional argument
                positional++;
            } else {
                std::cerr << "Unrecognized argument: " << argv[i] << std::endl;
                printArgs(argc, argv);
                exit(EXIT_FAILURE);
            }
        } else {
            type = it->second.type;
            output = it->second.store;

            it->second.required = false;
        }

        switch (type) {
            case TypeName::BOOL_EXPLICIT: {
                i++;

                checkArgBounds(argc, i, argv[i - 1]);

                if (!strcmp(argv[i], "true")) {
                    *(bool *)output = true;
                } else if (!strcmp(argv[i], "false")) {
                    *(bool *)output = false;
                } else {
                    std::cerr << "Invalid boolean value: " << argv[i]
                            << "for parameter " << argv[i - 1] << std::endl;
                    printArgs(argc, argv);
                    exit(EXIT_FAILURE);
                }

                break;
            }

            case TypeName::BOOL_IMPLICIT: {
                *(bool *)output = true;

                break;
            }

            case TypeName::INTEGER: {
                i++;

                checkArgBounds(argc, i, argv[i - 1]);

                int value = strtol(argv[i], nullptr, 10);
                if (value == 0) {
                    std::cerr << "Could not convert to integer value: \"" << argv[i]
                            << "\" requireg by parameter " << argv[i - 1] << std::endl;
                    printArgs(argc, argv);
                    exit(EXIT_FAILURE);
                }

                *(int *)output = value;

                break;
            }

            case TypeName::STRING: {
                i++;

                checkArgBounds(argc, i, argv[i - 1]);

                *(const char **)output = argv[i];

                break;
            }

            case TypeName::BOOL_POSITIONAL: {
                if (!strcmp(argv[i], "true")) {
                    *(bool *)output = true;
                } else if (!strcmp(argv[i], "false")) {
                    *(bool *)output = false;
                } else {
                    std::cerr << "Invalid boolean value: " << argv[i]
                            << "for parameter " << argv[i - 1] << std::endl;
                    printArgs(argc, argv);
                    exit(EXIT_FAILURE);
                }

                break;
            }

            case TypeName::INTEGER_POSITIONAL: {
                int value = strtol(argv[i], nullptr, 10);
                if (value == 0) {
                    std::cerr << "Could not convert to integer value: \"" << argv[i]
                            << "\" requireg by parameter " << argv[i - 1] << std::endl;
                    printArgs(argc, argv);
                    exit(EXIT_FAILURE);
                }

                *(int *)output = value;

                break;
            }

            case TypeName::STRING_POSITIONAL: {
                *(const char **)output = argv[i];

                break;
            }

        } // switch (TypeName)

    } // for arg in args

    bool error = false;

    for (auto pos : positionals) {
        if (pos.required) {
            std::cerr << "Missing required positional argument of type " << pos.type << std::endl;
            error = true;
        }
    }

    for (auto flag : flagsMap) {
        if (flag.second.required) {
            std::cerr << "Missing required argument \"" << flag.first << "\" of type " << flag.second.type << std::endl;
            error = true;
        }
    }

    if (error) {
        std::cerr << "Argument parsing failed" << std::endl;
        printArgs(argc, argv);
        exit(EXIT_FAILURE);
    }
}


// Lookup table for TypeName names
static const char *const typeNameNames[] = {
    "BOOL EXPLICIT",
    "BOOL IMPLICIT",
    "INTEGER",
    "STRING",
    "BOOL POSITIONAL",
    "INTEGER POSITIONAL",
    "STRING POSITIONAL"
};


std::ostream &operator<<(std::ostream &stream, TypeName typeName) {
    return stream << typeNameNames[(unsigned char)typeName];
}


std::ostream &operator<<(std::ostream &stream, const Parameter &parameter) {
    return stream << parameter.type << "\t\t"
                  << "required: " << (parameter.required ? "true" : "false") << '\t'
                  << parameter.description;
}

