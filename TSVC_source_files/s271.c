#include "globals.h"

int s271()
{

//	control flow
//	loop with singularity handling
		for (int i = 0; i < LEN; i++) {
			if (b[i] > (TYPE)0.) {
				a[i] += b[i] * c[i];
			}
		}
	return 0;
}

