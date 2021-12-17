#include "globals.h"

int s127()
{

//	induction variable recognition
//	induction variable with multiple increments

	int j;
		j = -1;
		for (int i = 0; i < LEN/2; i++) {
			j++;
			a[j] = b[i] + c[i] * d[i];
			j++;
			a[j] = b[i] + d[i] * e[i];
		}
	return 0;
}
