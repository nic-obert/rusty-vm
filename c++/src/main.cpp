#include "processor.hh"
#include "argparser.hh"


using namespace argparser;
using namespace processor;
using namespace error;


typedef struct Options {
    const char* file_name = nullptr;
    bool verbose = false;
    int stack_size = 1024;
    int video_size = 1024;
} Options;


static Parser createParser(Options& options) {
    Parser parser = Parser(
        4
        // TODO: add a description
    );

    parser.addStringPositional(
        &options.file_name, true,
        "name of the byte code file to execute"
    );

    parser.addBoolImplicit(
        "-v", &options.verbose, false,
        "verbose mode"
    );

    parser.addInteger(
        "-s", &options.stack_size, false,
        "stack size in bytes"
    );
    
    parser.addInteger(
        "-v", &options.video_size, false,
        "video size in bytes"
    );

    return parser;
}


int main(int argc, char** argv) {
    
    Options options;
    Parser parser = createParser(options);
    parser.parse(argc, argv);

    Byte* byteCode;
    size_t size = loadFileBytes(options.file_name, &byteCode);

    Processor processor(options.stack_size, options.video_size);
    ErrorCodes errorCode = processor.execute(byteCode, size, options.verbose);

    std::cout << "Program exited with code: " << errorCode << std::endl;

    return 0;
}
    
