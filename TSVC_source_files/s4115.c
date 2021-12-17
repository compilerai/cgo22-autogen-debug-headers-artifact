#include "globals.h"

int s4115(int* /*__restrict__*/ ip)
{

//	indirect addressing
//	sparse dot product
//	gather is required

	TYPE sum;
		sum = 0.;
		for (int i = 0; i < LEN; i++) {
			sum += a[i] * b[ip[i]];
		}
	temp = sum;
	return 0;
}

