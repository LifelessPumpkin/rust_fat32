#include <stdio.h>
#include <string.h>

#define BUF_LEN 1000


// ADVANCED technique, not really necessary, redundant code is fine.

// wrapped function to store initial file pointer before function call.

typedef struct {
    int size;
    char ** items;
} tokenlist;

FILE * fp;                

void read(tokenlist * tokens) {
    // calculate size
    fseek(fp , 0, SEEK_END);
    unsigned long size = ftell(fp);
    fseek(fp, 0, SEEK_SET);

    char buf[BUF_LEN];
    memset(buf, 0, BUF_LEN);
    fread(&buf, 1, size, fp);       // fp is at offset 29
    printf("String: %s\n", buf);
}

// Wrapper function that takes a function pointer as an argument
void wrapper(void (*func)(tokenlist * tokens), tokenlist * tokens) {
    unsigned long init_pos = ftell(fp);     // save old pos before moving fp around
    printf("Calling wrapped function...\n");
    func(tokens);                           // called wrapped function
    fseek(fp, init_pos, SEEK_SET);          // reset fp to old pos 
}

int main(int argc, char * argv[]) {
    if(argc != 2) {
        printf("./wrapper_example file\n");
        return -1;          // return error indicator if number of args is not 2.
    } 

    if((fp = fopen(argv[1], "r+")) == NULL) {   // r+ stands for read,write,append.
        printf("%s does not exist\n.", argv[1]);
        return -1;
    }
    tokenlist * tokens; // obviously we need to allocate memory to this, but just as an example.
    printf("Initial Position of File Pointer: %d\n", ftell(fp));
    wrapper(read, tokens);
    printf("Final Position of File Pointer: %d\n", ftell(fp));

    printf("\n\n");

    printf("Initial Position of File Pointer (without wrapper): %d\n", ftell(fp));
    read(tokens);
    printf("Final Position of File Pointer (without wrapper): %d\n", ftell(fp));
    
    return 0;
}