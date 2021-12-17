#include "globals.h"

int s253()
{

//	scalar and array expansion
//	scalar expansio assigned under if

	TYPE s;
		for (int i = 0; i < LEN; i++) {
			if (a[i] > b[i]) {
				s = a[i] - b[i] * d[i];
				c[i] += s;
				a[i] = s;
			}
		}
	return 0;
}
