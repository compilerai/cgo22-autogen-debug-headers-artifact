#include "globals.h"

int s3111()
{

//	reductions
//	conditional sum reduction

	TYPE sum;
		sum = 0.;
		for (int i = 0; i < LEN; i++) {
			if (a[i] > (TYPE)0.) {
				sum += a[i];
			}
		}
	temp = sum;
	return 0;
}

