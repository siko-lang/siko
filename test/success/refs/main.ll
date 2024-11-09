@.str_0 = private unnamed_addr constant [3 x i8] c"foo", align 1
%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_foo(ptr noundef %s, ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.siko_Tuple_, align 4
   call void @siko_Tuple_(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i1, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b0i28 = alloca %struct.siko_Tuple_, align 4
   %b0i27 = alloca %struct.siko_Tuple_, align 4
   %b0i24 = alloca ptr, align 8
   %b0i22 = alloca ptr, align 8
   %b0i21 = alloca %struct.String_String, align 8
   %b0i20 = alloca %struct.siko_Tuple_, align 4
   %b0i18 = alloca ptr, align 8
   %b0i17 = alloca %struct.String_String, align 8
   %b0i16 = alloca %struct.siko_Tuple_, align 4
   %b0i15 = alloca ptr, align 8
   %b0i14 = alloca %struct.String_String, align 8
   %ref2_2 = alloca ptr, align 8
   %b0i11 = alloca ptr, align 8
   %ref_1 = alloca ptr, align 8
   %b0i8 = alloca ptr, align 8
   %b0i6 = alloca ptr, align 8
   %b0i4 = alloca ptr, align 8
   %b0i3 = alloca %struct.String_String, align 8
   %s_0 = alloca %struct.String_String, align 8
   %b0i1 = alloca %struct.String_String, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.String_String, ptr %b0i1, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_b0i1_1, align 8
   %tmp_b0i1_2 = getelementptr inbounds %struct.String_String, ptr %b0i1, i32 0, i32 1
   store i64 3, ptr %tmp_b0i1_2, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %s_0, ptr align 8 %b0i1, i64 16, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i3, ptr align 8 %s_0, i64 16, i1 false)
   store ptr %b0i3, ptr %b0i4, align 8
   store ptr %b0i4, ptr %b0i6, align 8
   store ptr %b0i6, ptr %b0i8, align 8
   store ptr %b0i8, ptr %ref_1, align 8
   store ptr %ref_1, ptr %b0i11, align 8
   store ptr %b0i11, ptr %ref2_2, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i14, ptr align 8 %s_0, i64 16, i1 false)
   store ptr %b0i14, ptr %b0i15, align 8
   %tmp_b0i15_0 = load ptr, ptr %b0i15, align 8
   call void @Main_foo(ptr %tmp_b0i15_0, ptr %b0i16)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i17, ptr align 8 %s_0, i64 16, i1 false)
   store ptr %b0i17, ptr %b0i18, align 8
   %tmp_b0i18_0 = load ptr, ptr %b0i18, align 8
   call void @Main_foo(ptr %tmp_b0i18_0, ptr %b0i20)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %b0i21, ptr align 8 %s_0, i64 16, i1 false)
   store ptr %b0i21, ptr %b0i22, align 8
   store ptr %b0i22, ptr %b0i24, align 8
   %tmp_b0i24_0 = load ptr, ptr %b0i24, align 8
   call void @Main_foo(ptr %tmp_b0i24_0, ptr %b0i27)
   call void @siko_Tuple_(ptr %b0i28)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i28, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


