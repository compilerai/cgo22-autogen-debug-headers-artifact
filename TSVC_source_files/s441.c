#include "globals.h"

int s441()
 {
 
 //	non-logical if's
 //	arithmetic if
 
 		for (int i = 0; i < LEN; i++) {
 			if (d[i] < (TYPE)0.) {
 				a[i] += b[i] * c[i];
 			} else if (d[i] == (TYPE)0.) {
 				a[i] += b[i] * b[i];
 			} else {
 				a[i] += c[i] * c[i];
 			}
 		}
 	return 0;
 }

