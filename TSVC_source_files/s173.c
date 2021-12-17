#include "globals.h"

int s173()
{
//	symbolics
//	expression in loop bounds and subscripts

	int k = LEN/2;
		for (int i = 0; i < LEN/2; i++) {
			a[i+k] = a[i] + b[i];
		}
	return 0;
}

