#include "globals.h"

int s171(int inc)
{

//	symbolics
//	symbolic dependence tests

		for (int i = 0; i < LEN; i++) {
			a[i * inc] += b[i];
		}
	return 0;
}
