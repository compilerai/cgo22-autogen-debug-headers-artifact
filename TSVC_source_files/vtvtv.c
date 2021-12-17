#include "globals.h"

int vtvtv()
{

//	control loops
//	vector times vector times vector

		for (int i = 0; i < LEN; i++) {
			a[i] = a[i] * b[i] * c[i];
		}
	return 0;
}

