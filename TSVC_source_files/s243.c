#include "globals.h"

int s243()
{

//	node splitting
//	false dependence cycle breaking

		for (int i = 0; i < LEN-1; i++) {
			a[i] = b[i] + c[i  ] * d[i];
			b[i] = a[i] + d[i  ] * e[i];
			a[i] = b[i] + a[i+1] * d[i];
		}
	return 0;
}

