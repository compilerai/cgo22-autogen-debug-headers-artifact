#include "globals.h"

int s319()
{

//	reductions
//	coupled reductions

	TYPE sum;
		sum = 0.;
		for (int i = 0; i < LEN; i++) {
			a[i] = c[i] + d[i];
			sum += a[i];
			b[i] = c[i] + e[i];
			sum += b[i];
		}
	temp = sum;
	return 0;
}

