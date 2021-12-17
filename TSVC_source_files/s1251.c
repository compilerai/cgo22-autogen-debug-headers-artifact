#include "globals.h"

int s1251()
{

//	scalar and array expansion
//	scalar expansion

	TYPE s;
		for (int i = 0; i < LEN; i++) {
			s = b[i]+c[i];
			b[i] = a[i]+d[i];
			a[i] = s*e[i];
		}
	return 0;
}
