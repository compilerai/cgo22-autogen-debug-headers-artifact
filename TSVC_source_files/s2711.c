#include "globals.h"

int s2711()
{

//	control flow
//	semantic if removal

		for (int i = 0; i < LEN; i++) {
			if (b[i] != (TYPE)0.0) {
				a[i] += b[i] * c[i];
			}
		}
	return 0;
}

