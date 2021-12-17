#include "globals.h"

int s000()
{

//	linear dependence testing
//	no dependence - vectorizable

		for (int i = 0; i < lll; i++) {
			X[i] = Y[i] + val;
		}
	return 0;
}


