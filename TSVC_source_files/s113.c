#include "globals.h"

int s113()
{

//	linear dependence testing
//	a(i)=a(1) but no actual dependence cycle

		for (int i = 1; i < LEN; i++) {
			a[i] = a[0] + b[i];
		}
	return 0;
}
