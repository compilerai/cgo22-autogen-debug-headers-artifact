#include "globals.h"

int s2710( TYPE x)
{

//	control flow
//	scalar and vector ifs

		for (int i = 0; i < LEN; i++) {
			if (a[i] > b[i]) {
				a[i] += b[i] * d[i];
				if (LEN > 10) {
					c[i] += d[i] * d[i];
				} else {
					c[i] = d[i] * e[i] + (TYPE)1.;
				}
			} else {
				b[i] = a[i] + e[i] * e[i];
				if (x > (TYPE)0.) {
					c[i] = a[i] + d[i] * d[i];
				} else {
					c[i] += e[i] * e[i];
				}
			}
		}
	return 0;
}

