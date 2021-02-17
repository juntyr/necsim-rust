// Adapted from https://www.pcg-random.org/posts/how-to-test-with-testu01.html

// gcc -std=c99 -Wall -O3 -o test test.c -Iinclude -Llib -ltestu01 -lprobdist -lmylib -lm

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

#include "TestU01.h"

char* gen_name = "hello-world";

static uint64_t state = 0;
static uint64_t weyl = 0;

uint64_t diffuse (uint64_t x)
{
    x *= 0x6eed0e9da4d94a4f;
    
    uint64_t a = x >> 32;
    uint64_t b = x >> 60;
    
    x ^= (a >> b);
    
    return x * 0x6eed0e9da4d94a4f;
}

uint64_t gen64 (void)
{
    state = diffuse(state) + (weyl += 0xb5ad4eceda1ce2a9);
    
    return state;
}

inline uint32_t rev32(uint32_t v)
{
    // https://graphics.stanford.edu/~seander/bithacks.html
    // swap odd and even bits
    v = ((v >> 1) & 0x55555555) | ((v & 0x55555555) << 1);
    // swap consecutive pairs
    v = ((v >> 2) & 0x33333333) | ((v & 0x33333333) << 2);
    // swap nibbles ...
    v = ((v >> 4) & 0x0F0F0F0F) | ((v & 0x0F0F0F0F) << 4);
    // swap bytes
    v = ((v >> 8) & 0x00FF00FF) | ((v & 0x00FF00FF) << 8);
    // swap 2-byte-long pairs
    v = ( v >> 16             ) | ( v               << 16);
    return v;
}

inline uint32_t high32(uint64_t v)
{
    return (v >> 32) & 0x00000000FFFFFFFF;
}

inline uint32_t low32(uint64_t v)
{
    return v & 0x00000000FFFFFFFF;
}

uint32_t gen32_high()
{
    return high32(gen64());
}

uint32_t gen32_high_rev()
{
    return rev32(high32(gen64()));
}

uint32_t gen32_low()
{
    return low32(gen64());
}

uint32_t gen32_low_rev()
{
    return rev32(low32(gen64()));
}

const char* progname;

void usage()
{
    printf("%s: [-v] [-r]\n", progname);
    exit(1);
}

int main (int argc, char** argv)
{
    progname = argv[0];

    // Config options for TestU01
    swrite_Basic = FALSE;  // Turn of TestU01 verbosity by default
                           // reenable by -v option.

    // Config options for generator output
    bool reverseBits = false;
    bool highBits = false;

    // Config options for tests
    bool testSmallCrush = false;
    bool testCrush = false;
    bool testBigCrush = false;
    bool testLinComp = false;

    // Handle command-line option switches
    while (1) {
        --argc; ++argv;
        if ((argc == 0) || (argv[0][0] != '-'))
            break;
        if ((argv[0][1]=='\0') || (argv[0][2]!='\0'))
            usage();
        switch(argv[0][1]) {
        case 'r':
            reverseBits = true;
            break;      
        case 'h':
            highBits = true;
            break;      
        case 's':
            testSmallCrush = true;
            break;
        case 'm':
            testCrush = true;
            break;
        case 'b':
            testBigCrush = true;
            break;
        case 'l':
            testLinComp = true;
            break;
        case 'v':
            swrite_Basic = TRUE;
            break;
        default:
            usage();
        }
    }

    // Name of the generator

    printf("Testing %s%s%s:\n", gen_name, highBits ? " [High bits]" : " [Low bits]", reverseBits ? " [Reversed]" : "");

    // Determine a default test if need be

    if (!(testSmallCrush || testCrush || testBigCrush || testLinComp)) {
        testCrush = true;
    }
    
    fflush(stdout);

    // Create a generator for TestU01.

    unif01_Gen* gen =
        unif01_CreateExternGenBits(gen_name,
          reverseBits ? (highBits ? gen32_high_rev : gen32_low_rev)
                      : (highBits ? gen32_high     : gen32_low));

    // Run tests.

    if (testSmallCrush) {
        printf("Testing Small Crush ...\n");
        bbattery_SmallCrush(gen);
        fflush(stdout);
    }
    if (testCrush) {
        printf("Testing Crush ...\n");
        bbattery_Crush(gen);
        fflush(stdout);
    }
    if (testBigCrush) {
        printf("Testing Bif Crush ...\n");
        bbattery_BigCrush(gen);
        fflush(stdout);
    }
    if (testLinComp) {
        printf("Testing Lin Comp ...\n");
        
        scomp_Res* res = scomp_CreateRes();
        swrite_Basic = TRUE;
        
        scomp_LinearComp(gen, res, 1, 250, 0, 1);
        scomp_LinearComp(gen, res, 1, 500, 0, 1);
        scomp_LinearComp(gen, res, 1, 1000, 0, 1);
        scomp_LinearComp(gen, res, 1, 5000, 0, 1);
        scomp_LinearComp(gen, res, 1, 25000, 0, 1);
        scomp_LinearComp(gen, res, 1, 50000, 0, 1);
        scomp_LinearComp(gen, res, 1, 75000, 0, 1);
        
        scomp_DeleteRes(res);
        fflush(stdout);
    }

    // Clean up.

    unif01_DeleteExternGenBits(gen);

    return 0;
}
