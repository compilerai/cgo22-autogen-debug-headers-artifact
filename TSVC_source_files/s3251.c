#include "globals.h"

int s3251()
{

//	scalar and array expansion
//	scalar expansion

		for (int i = 0; i < LEN-1; i++){
			a[i+1] = b[i]+c[i];
			b[i]   = c[i]*e[i];
			d[i]   = a[i]*e[i];
		}
	return 0;
}

