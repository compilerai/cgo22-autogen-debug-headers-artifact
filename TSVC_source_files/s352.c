#include "globals.h"

int s352()
{

//	loop rerolling
//	unrolled dot product

	TYPE dot;
		dot = (TYPE)0.;
	  int i1=0;
		for (int i = 0; i < LEN; i += 5) {
			i1++;
			dot = dot + a[i] * b[i] + a[i + 1] * b[i + 1] + a[i + 2]
				* b[i + 2] + a[i + 3] * b[i + 3] + a[i + 4] * b[i + 4];
		}
	temp = dot;
	return 0;
}

