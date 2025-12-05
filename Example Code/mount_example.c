#include <stdio.h>
#include <string.h>

#define BUF_LEN 1000
/*
*  Readin provided file that contains a section of a directory cluster.
* 
*/

FILE * fp;                  // file pointers
FILE * fp1;
FILE * fp2;

int main(int argc, char * argv[]) {
    char buf[BUF_LEN];
    
    if(argc != 2) {
        printf("invalid number of arguments.\n");
        return -1;          // return error indicator if number of args is not 2.
    } 

    if((fp = fopen(argv[1], "r+")) == NULL) {   // r+ stands for read,write,append.
        printf("%s does not exist\n.", argv[1]);
        return -1;
    }

    // you can have multiple file pointers that point to the same pointer.
    // however you have to use fopen.
    
    fseek(fp, 0, SEEK_END);
    printf("size of file: %d\n", ftell(fp));
    unsigned int size = ftell(fp);

    // we can also navigate backwards or forwards from current position with SEEK_CUR. 
    fseek(fp, -7, SEEK_CUR);

    fp1 = fopen(argv[1], "r+");  // and write to them
    fread(buf, ftell(fp), 1, fp1); // read from beginning of file from different file pointer
    printf("%s\n", buf);
    
    fseek(fp, 6, SEEK_SET);        // point the 1st file pointer to index 6.
    memset(buf, 0, BUF_LEN);       // sets entire buffer to null characters.    
    fread(buf, 8, 1, fp);          // reads 8 bytes from index 6 of first file pointer.

    printf("%s\n", buf);           


    fp2 = fopen(argv[1], "r+");
    // reset fp to initial position in file.
    fseek(fp, 0, SEEK_SET);

    // write from fp2, at position at offset 6
    fseek(fp2, 6, SEEK_SET);
    memset(buf, 0, BUF_LEN);
    strcpy(buf, "I WROTE HERE!");
    fwrite(&buf, strlen(buf), 1, fp2);

    // reading new write from fp
    fflush(fp2);
    fflush(fp1);
    fflush(fp);
    fread(buf,size, 1, fp);

    printf("%s\n", buf);
    fclose(fp);
    fclose(fp1);    

    return 0;
}