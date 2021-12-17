#include "globals.h"

int vif()
{

//	control loops
//	vector if

		for (int i = 0; i < LEN; i++) {
			if (b[i] > (TYPE)0.) {
				a[i] = b[i];
			}
		}
	return 0;
}
