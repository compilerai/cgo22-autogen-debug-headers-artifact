#include "globals.h"

int s351()
{

//	loop rerolling
//	unrolled saxpy

	TYPE alpha = c[0];
	int i1 = 0;
		for (int i = 0; i < LEN; i = i+5) {
		  i1++;
			a[i] += alpha * b[i];
			a[i + 1] += alpha * b[i + 1];
			a[i + 2] += alpha * b[i + 2];
			a[i + 3] += alpha * b[i + 3];
			a[i + 4] += alpha * b[i + 4];
		}
	return 0;
}

