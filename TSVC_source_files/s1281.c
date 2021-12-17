#include "globals.h"

int s1281()
{

//	crossing thresholds
//	index set splitting
//	reverse data access

	TYPE x;
		for (int i = 0; i < LEN; i++) {
			x = b[i]*c[i]+a[i]*d[i]+e[i];
			a[i] = x-(TYPE)1.0;
			b[i] = x;
		}
	return 0;
}
