#include "globals.h"

int s128()
{

//	induction variables
//	coupled induction variables
//	jump in data access

	int j, k;
		j = -1;
		for (int i = 0; i < LEN/2; i++) {
			k = j + 1;
			a[i] = b[k] - d[i];
			j = k + 1;
			b[k] = a[i] + c[k];
		}
	return 0;
}
