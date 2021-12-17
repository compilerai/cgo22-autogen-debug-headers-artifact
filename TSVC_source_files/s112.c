#include "globals.h"

int s112()
{

//	linear dependence testing
//	loop reversal

		for (int i = LEN - 2; i >= 0; i--) {
			a[i+1] = a[i] + b[i];
		}
	return 0;
}

