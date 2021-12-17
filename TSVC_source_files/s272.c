#include "globals.h"

int s272(TYPE t)
{

//	control flow
//	loop with independent conditional

		for (int i = 0; i < LEN; i++) {
			if (e[i] >= t) {
				a[i] += c[i] * d[i];
				b[i] += c[i] * c[i];
			}
		}
	return 0;
}
