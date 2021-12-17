#include "globals.h"

int s116()
{

//	linear dependence testing
	int i1 = 0;  
	for (int i = 0; i < LEN-5 ; i += 5, i1++) {
		a[i] = a[i + 1] * a[i];
		a[i + 1] = a[i + 2] * a[i + 1];
		a[i + 2] = a[i + 3] * a[i + 2];
		a[i + 3] = a[i + 4] * a[i + 3];
		a[i + 4] = a[i + 5] * a[i + 4];
	}
	return 0;
}
