#include "globals.h"

int s131()
{
//	global data flow analysis
//	forward substitution

	int m  = 2;
		for (int i = 0; i < LEN - 2; i++) {
			a[i] = a[i + m] + b[i];
		}
	return 0;
}
