
/* TSVC functions vectorized by either of GCC, CLANG or ICC and not demonstrated by any prior work */

#include "globals.h"

#define TYPE int

#define lll LEN


TYPE val = 1;
__attribute__ ((aligned(16))) TYPE X[lll],Y[lll],Z[lll],U[lll],V[lll];

TYPE array[LEN2*LEN2] __attribute__((aligned(16)));

TYPE x[LEN] __attribute__((aligned(16)));
TYPE temp, s;

__attribute__((aligned(16))) TYPE a[LEN],b[LEN],c[LEN],d[LEN],e[LEN],
                                   aa[LEN2][LEN2],bb[LEN2][LEN2],cc[LEN2][LEN2],tt[LEN2][LEN2];


TYPE indx[LEN] __attribute__((aligned(16)));


TYPE* /*__restrict__*/ xx;
TYPE* yy;
TYPE arr[LEN];








int val_s;






extern int s000();
int main()
{
  s000();
  return 0;
}
