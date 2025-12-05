#include <stdio.h>
#include <string.h>

// assume the correct values for "LONGFILE" Dir_Name = "LONGFILE"; 
// DIR_FstClusLo = "0x01"; DIR_FstClusHi = "0x00"; DIR_Filesize = "220022";
// assume this following format for a file entry in some filesystem.

typedef struct __attribute__((packed)){
    unsigned char DIR_Name[8];             // 8 bytes (1 byte read), this representation is fine as it is character string.
    unsigned short DIR_FstClusHi;          // 2 bytes (2 byte read), this representation is in high word of first clus
    unsigned int DIR_FileSize;             // 4 bytes (4 byte read), this representation is size of file.
    unsigned short DIR_FstClusLo;          // 2 bytes (2 byte read), this representation is in low word of first clus.
} DIR;

int main() {
    FILE * fp = fopen("entries", "r+");
    DIR entry;

    // this is how we read in blocks of data.
    fread(&entry, sizeof(DIR), 1, fp); // format for reading in a struct
    
    printf("Offset: %d\n", ftell(fp)); // implies we read in 16 bytes and our pointer is now at position 16 (starting index is 0).

    printf("\nReading value of entry from struct:\n");
    printf("DIR_Name = %s\n", entry.DIR_Name); // you might need to append a null character to end of DIR_Name if necessary.
    
    printf("DIR_FstClusLo = %d\n", entry.DIR_FstClusLo);
    printf("DIR_FstClusHi = %d\n", entry.DIR_FstClusHi);

    // we use the actual equation to get the actual cluster location.
    unsigned int cluster = entry.DIR_FstClusLo + (65536 * entry.DIR_FstClusHi);

    printf("\nFirst Cluster Location of %s: %d\n", entry.DIR_Name, cluster);
    printf("DIR_FileSize = %d\n", entry.DIR_FileSize);

    // you would need to fseek with fstcluster using the formula in bytes.
    printf("We will now write to file of size DIR entry at offset 48 bytes. (uncomment in code)\n");
    fseek(fp, 48, SEEK_SET);
    printf("After fseek(), Offset: %d\n\n", ftell(fp));

    // check "entries" file in hexedit before uncommenting and check after!
   
    // assume our dir entry value we want to write are 
    // DIR_Name = "NEWFILE"
    // DIR_Cluster = 65537
    // DIR_FileSize = 41002;

 // write command, uncomment here. 
#if 0
    DIR _entry; // entry to write
    strcpy(_entry.DIR_Name, "NEWFILE\0");

    // we do not worry about little endian here, just write values as they are and everything will be taken care of.
    //DIR FstClus = 0x00010001 = 65537.
    _entry.DIR_FstClusLo = 0x0001;  // does not need to be in hexadecimal, can be in decimal
    _entry.DIR_FstClusHi = 0x0001;
    _entry.DIR_FileSize = 41002;

    // we write at offset 48 as we define it to be there.
    printf("%d\n", fwrite(&_entry, sizeof(DIR), 1, fp)); // format should look something like this.
    
    printf("We wrote to file our entry. Here is our offset: %d\n", ftell(fp));
    fseek(fp ,-sizeof(DIR), SEEK_CUR);
    printf("We need to go back the sizeof(DIR) bytes to read our changes.\nHere is our offset: %d\n", ftell(fp)); 
    DIR entry_read;
    
    fread(&entry_read, sizeof(DIR), 1, fp);

    unsigned int  fst_clus = entry_read.DIR_FstClusLo + (65536 * entry_read.DIR_FstClusHi);
    printf("Name: %s\nFirst Clus: %d\nFileSize: %d\n", entry_read.DIR_Name, fst_clus, entry_read.DIR_FileSize);    
#endif

    fclose(fp);
    return 0;
}