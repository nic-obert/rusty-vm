#include "processor.hh"
#include "argparser.hh"


using namespace argparser;


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

    


}
    
