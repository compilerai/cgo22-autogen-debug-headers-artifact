#include "globals.h"

int vdotr()
{

//	control loops
//	vector dot product reduction

	TYPE dot;
		dot = 0.;
		for (int i = 0; i < LEN; i++) {
			dot += a[i] * b[i];
		}
	temp = dot;
	return 0;
}

