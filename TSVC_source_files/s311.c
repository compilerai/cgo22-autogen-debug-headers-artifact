#include "globals.h"

int s311()
{

//	reductions
//	sum reduction

	TYPE sum;
		sum = (TYPE)0.;
		for (int i = 0; i < LEN; i++) {
			sum += a[i];
		}
		temp = sum;
	return 0;
}
