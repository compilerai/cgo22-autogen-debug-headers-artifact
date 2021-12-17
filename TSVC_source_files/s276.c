#include "globals.h"

int s276()
{

//	control flow
//	if test using loop index

	int mid = (LEN/2);
		for (int i = 0; i < LEN; i++) {
			if (i+1 < mid) {
				a[i] += b[i] * c[i];
			} else {
				a[i] += b[i] * d[i];
			}
		}
	return 0;
}

