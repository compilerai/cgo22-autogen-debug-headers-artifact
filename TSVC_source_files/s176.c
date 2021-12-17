#include "globals.h"

int s176()
{

//	symbolics
//	convolution
	int m = LEN/2;
		for (int j = 0; j < (LEN/2); j++) {
			for (int i = 0; i < m; i++) {
				a[i] += b[i+m-j-1] * c[j];
			}
//			DBG(__LINE__);
		}
	return 0;
}

