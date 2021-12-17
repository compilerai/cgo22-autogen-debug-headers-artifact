#include "globals.h"

int s273()
{

//	control flow
//	simple loop with dependent conditional

		for (int i = 0; i < LEN; i++) {
			a[i] += d[i] * e[i];
			if (a[i] < (TYPE)0.)
				b[i] += d[i] * e[i];
			c[i] += a[i] * d[i];
		}
	return 0;
}
