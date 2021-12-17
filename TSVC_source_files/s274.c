#include "globals.h"

int s274()
{

//	control flow
//	complex loop with dependent conditional

		for (int i = 0; i < LEN; i++) {
			a[i] = c[i] + e[i] * d[i];
			if (a[i] > (TYPE)0.) {
				b[i] = a[i] + b[i];
			} else {
				a[i] = d[i] * e[i];
			}
		}
	return 0;
}

