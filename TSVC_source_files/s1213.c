#include "globals.h"

int s1213()
{

//	statement reordering
//	dependency needing temporary

		for (int i = 1; i < LEN-1; i++) {
			a[i] = b[i-1]+c[i];
			b[i] = a[i+1]*d[i];
		}
	return 0;
}
