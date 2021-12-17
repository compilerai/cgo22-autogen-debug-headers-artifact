#include "globals.h"

int s254()
{

  //	scalar and array expansion
  //	carry around variable

	TYPE x;
	x = b[LEN-1];
	for (int i = 0; i < LEN; i++) {
		a[i] = (b[i] + x) * (TYPE).5;
//		a[i] = (b[i] + x) / 2;
		x = b[i];
	}
	return 0;
}

