#include "globals.h"

int s1161()
{

//	control flow
//	tests for recognition of loop independent dependences
//	between statements in mutually exclusive regions.

		for (int i = 0; i < LEN-1; ++i) {
			if (c[i] < (TYPE)0.) {
				goto L20;
			}
			a[i] = c[i] + d[i] * e[i];
			goto L10;
L20:
			b[i] = a[i] + d[i] * d[i];
L10:
			;
		}
	return 0;
}
