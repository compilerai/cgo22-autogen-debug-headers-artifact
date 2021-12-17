#include "globals.h"

#ifndef icc_compiler
int s331()
{

//	search loops
//	if to last-1

	int j;
	TYPE chksum;
		j = -1;
		for (int i = 0; i < LEN; i++) {
			if (a[i] < (TYPE)0.) {
				j = i;
			}
		}
		chksum = (TYPE) j;
	temp = j+1;
	return 0;
}
#endif

