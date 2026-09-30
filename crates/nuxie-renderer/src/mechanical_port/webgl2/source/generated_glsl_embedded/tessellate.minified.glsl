#define Yh 10
#ifdef CB
g1(h0) I(0,f,MD);I(1,f,ND);I(2,f,VC);
#ifdef Z9
I(3,uint,IE);I(4,uint,JE);I(5,uint,KE);I(6,uint,LE);
#else
I(3,R,UB);
#endif
h1
#endif
r2 I0 W(0,f,B6);I0 W(1,f,C6);I0 W(2,f,S4);I0 W(3,S,T4);W2 W(4,uint,G7);i2
#ifdef CB
Y3 h6(h3,j7,YC);Z3 g4(j7,ca) G4 N4(Zc,Ig,OB);N4(ad,Jg,ID);H4 A1(FG,h0,F,A,q){J(q,F,MD,f);J(q,F,ND,f);J(q,F,VC,f);
#ifdef Z9
J(q,F,IE,uint);J(q,F,JE,uint);J(q,F,KE,uint);J(q,F,LE,uint);R UB=R(IE,JE,KE,LE);
#else
J(q,F,UB,R);
#endif
V(B6,f);V(C6,f);V(S4,f);V(T4,S);V(G7,uint);c x0=MD.xy;c B0=MD.zw;c F0=ND.xy;c M0=ND.zw;bool fe=A<4;float y=fe?VC.z:VC.w;int ib=int(fe?UB.x:UB.y);
#ifdef da
int ge=ib<<16;if(UB.z==0xffffffffu){--ge;}float a9=float(ge>>16);
#else
float a9=float(ib<<16>>16);
#endif
float c9=float(ib>>16);c v2=c((A&1)==0?a9:c9,(A&2)==0?y+1.:y);if((c9-a9)*j.Bd<.0){v2.y=2.*y+1.-v2.y;}uint V2=UB.z&0x3ffu;uint he=(UB.z>>10)&0x3ffu;uint o2=UB.z>>20;uint m0=UB.w;uint E8=m0&Vc;uint o0=E8>0u?L0(ID,max(E8,1u)-1u).z:0u;R P4=o0!=0u?L0(OB,o0*4u+1u):R(0u,0u,0u,0u);float O2=uintBitsToFloat(P4.z);float P2=uintBitsToFloat(P4.w);if(P2!=.0&&O2==.0){float ie;float Zh=Cf(x0,B0,F0,M0,ie);float jb=P2*(1./qa);float ai=xf(x0,B0,F0,M0,ie,jb);float H7=1.-ai*(1./X3);float bi=dot(M0-x0,M0-x0)/(jb*jb);float ci=(bi-1.)*.5;H7=min(H7,ci);H7=min(H7,.99);float di=.5*H7;float x=zc(di)*-2.+1.;float je=k8(x*P2,Zh);f ke=mix(x0.xyxy,M0.xyxy,f(1./3.,1./3.,2./3.,2./3.));B0=mix(B0,ke.xy,je);F0=mix(F0,ke.zw,je);}if((m0&dg)!=0u){Y T8=L1(uintBitsToFloat(L0(OB,o0*4u)));c le=N0(T8,-2.*B0+F0+x0);c me=N0(T8,-2.*F0+M0+B0);float n1=max(dot(le,le),dot(me,me));float S3=max(ceil(sqrt(.75*4.*sqrt(n1))),1.);V2=min(uint(S3),V2);}uint d9=V2+he+o2-1u;Y m2=W9(x0,B0,F0,M0);float m1=acos(g8(m2[0],m2[1]));float v4=m1/float(he);float kb=determinant(Y(F0-x0,M0-B0));if(kb==.0) kb=determinant(m2);if(kb<.0) v4=-v4;B6=f(x0,B0);C6=f(F0,M0);S4=f(float(d9)-abs(c9-v2.x),float(d9),(o2<<10)|V2,v4);T4.xy=VC.xy;if(o2>1u){Y lb=Y(m2[1],VC.xy);float ei=acos(g8(lb[0],lb[1]));float ne=float(o2);if((m0&(J3|y8))==(i7|y8)){ne-=2.;}float mb=ei/ne;if(determinant(lb)<.0) mb=-mb;T4.z=mb;}if(c9<a9){m0|=f4;}G7=m0;f X=o8(v2,2./ag,j.Bd);
#ifdef SC
X.y=-X.y;
#endif
c0(B6);c0(C6);c0(S4);c0(T4);c0(G7);B1(X);}
#endif
#ifdef EB
H3 I3 f3(R,GG){r(B6,f);r(C6,f);r(S4,f);r(T4,S);r(G7,uint);c x0=B6.xy;c B0=B6.zw;c F0=C6.xy;c M0=C6.zw;Y m2=W9(x0,B0,F0,M0);float fi=max(floor(S4.x),.0);float d9=S4.y;uint oe=uint(S4.z);float V2=float(oe&0x3ffu);float o2=float(oe>>10);float v4=S4.w;uint m0=G7;float U4=d9-o2;float W1=fi;if(W1<=U4){m0&=~J3;}else{x0=B0=F0=M0;m2=Y(m2[1],T4.xy);V2=1.;W1-=U4;U4=o2;v4=T4.z;if((m0&J3)>i7){if(W1<2.5) m0|=Sc;if(W1>1.5&&W1<3.5) m0|=Tc;}else if((m0&y8)!=0u||(m0&J3)==x8){U4-=2.;--W1;}m0|=v4<.0?z8:Uc;}c H5;float m1=.0;if(W1==.0||W1==U4||(m0&J3)>i7){bool H8=W1<U4*.5;H5=H8?x0:M0;m1=Bc(H8?m2[0]:m2[1]);}else if((m0&Rc)!=0u){H5=x0;if(W1>=float(oa/2u)) H5=B0;if(W1>=float(oa*3u/4u)) H5=F0;if(W1>=float(oa*7u/8u)) H5=T4.xy;}else{float v1,I5;if(V2==U4){v1=W1/V2;I5=.0;}else{c B,H,l2=B0-x0;c P6=M0-x0;c h8=F0-B0;H=h8-l2;B=-3.*h8+P6;c gi=H*(V2*2.);c R6=l2*(V2*V2);float e9=.0;float hi=min(V2-1.,W1);c nb=normalize(m2[0]);float ii=-abs(v4);float ji=(1.+W1)*abs(v4);for(int ob=Yh-1;ob>=0;--ob){float I7=e9+exp2(float(ob));if(I7<=hi){c pb=I7*B+gi;pb=I7*pb+R6;float ki=dot(normalize(pb),nb);float qb=I7*ii+ji;qb=min(qb,X3);if(ki>=cos(qb)) e9=I7;}}float li=e9/V2;float pe=W1-e9;float f9=acos(clamp(nb.x,-1.,1.));f9=nb.y>=.0?f9:-f9;m1=pe*v4+f9;c M2=c(sin(m1),-cos(m1));float l=dot(M2,B),g9=dot(M2,H),J1=dot(M2,l2);float mi=max(g9*g9-l*J1,.0);float y2=sqrt(mi);if(g9>.0) y2=-y2;y2-=g9;float qe=-.5*y2*l;c rb=(abs(y2*y2+qe)<abs(l*J1+qe))?c(y2,l):c(J1,y2);I5=(rb.y!=.0)?rb.x/rb.y:.0;I5=clamp(I5,.0,1.);if(pe==.0) I5=.0;v1=max(li,I5);}c ni=a6(x0,B0,v1);c re=a6(B0,F0,v1);c oi=a6(F0,M0,v1);c se=a6(ni,re,v1);c te=a6(re,oi,v1);H5=a6(se,te,v1);if(v1!=I5) m1=Bc(te-se);}R J7;J7.xy=floatBitsToUint(H5);if((m0&J3)==x8){J7.z=(uint(U4)<<16)|uint(W1);}else{uint pi=uint(int(round(m1*(65536./p8))))&0xffffu;uint ue=0u;if((m0&J3)>i7){float qi=clamp(g8(m2[0],m2[1]),-1.,1.);ue=uint(round(sqrt((1.+qi)*.5)*65535.));}J7.z=(pi<<16)|ue;}J7.w=m0;N2(J7);}
#endif
