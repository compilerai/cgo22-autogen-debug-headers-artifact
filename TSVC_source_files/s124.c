#include "globals.h"

int s124()
 {
 
 //	induction variable recognition
 //	induction variable under both sides of if (same value)
 
 	int j;
 		j = -1;
 		for (int i = 0; i < LEN; i++) {
 			if (b[i] > (TYPE)0.) {
 				j++;
 				a[j] = b[i] + d[i] * e[i];
 			} else {
 				j++;
 				a[j] = c[i] + d[i] * e[i];
 			}
 		}
 	return 0;
 }
