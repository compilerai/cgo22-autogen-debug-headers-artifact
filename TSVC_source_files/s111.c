#include "globals.h"

int s111()
{

//	linear dependence testing
//	no dependence - vectorizable

		for (int i = 1; i < LEN; i += 2) {
			a[i] = a[i - 1] + b[i];
		}
	return 0;
}


